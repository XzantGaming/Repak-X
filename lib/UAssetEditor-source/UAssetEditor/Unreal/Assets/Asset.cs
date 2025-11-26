using System.Data;
using System.Reflection;
using Newtonsoft.Json;
using Serilog;
using UAssetEditor.Summaries;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Objects;
using UAssetEditor.Unreal.Properties.Structs;
using UAssetEditor.Unreal.Properties.Unversioned;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Classes.Containers;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Unreal.Properties;
using UAssetEditor.Unreal.Properties.Types;
using UAssetEditor.Unreal.Readers;
using UAssetEditor.Unreal.Versioning;
using UAssetEditor.Utils;
using UsmapDotNet;

namespace UAssetEditor.Unreal.Assets;

public class ObjectContainer : Container<UObject>
{
    public List<UObject> Objects => Items;

    public override int GetIndex(string str)
    {
        return Items.FindIndex(x => x.Name == str);
    }
    
    public UObject? this[string name]
    {
        get
        {
            foreach (var property in this)
            {
                if (property.Name != name)
                    continue;

                return property;
            }

            return null;
        }
        set
        {
            for (int i = 0; i < Length; i++)
            {
                var property = this[i];
                if (property.Name != name)
                    continue;

                if (value is null)
                    throw new NoNullAllowedException("Cannot set property to null");
                
                this[i] = value;
            }
        }
    }

    public ObjectContainer(List<UObject> objects) : base(objects)
    { }
    
    public ObjectContainer() : base(new List<UObject>())
    { }
}

public abstract class Asset : Reader
{
    public UnrealFileSystem? System;
    public UnrealFileReader? Reader;
    
    public string Name { get; set; }
    public Usmap? Mappings;
    public NameMapContainer NameMap;
    public EPackageFlags Flags;

    public EGame Game = EGame.GAME_UE5_LATEST; // TODO implement
    public FPackageFileVersion FileVersion = FPackageFileVersion.AUTO;
    
    public bool HasUnversionedProperties => Flags.HasFlag(EPackageFlags.PKG_UnversionedProperties);

    public StructureContainer DefinedStructures = new();

    public ObjectContainer Exports = new();
    
    // Export objects to load after everything has been read
    public List<ResolvedExportObject> ExportObjectsToLoad = new();

    public Asset(byte[] data, UnrealFileSystem? system = null, UnrealFileReader? reader = null) : base(data)
    {
        System = system;
        Reader = reader;
    }

    public UObject? this[string name] => Exports[name];
    
    public Asset(string path) : this(File.ReadAllBytes(path))
    { }

    public void CheckMappings()
    {
        if (Mappings is null)
            throw new NoNullAllowedException("Mappings cannot be null");
    }

    public void SetVersion(EUnrealEngineObjectUE4Version version) =>
        FileVersion = FPackageFileVersion.CreateUE4Version(version);

    public void SetVersion(EUnrealEngineObjectUE5Version version) =>
        FileVersion = FPackageFileVersion.CreateUE5Version(version);

    /// <summary>
    /// Read the entirety of this asset
    /// </summary>
    public abstract void ReadAll();
    public abstract uint ReadHeader();
    
    // TODO eventually redo when I add pak assets (not unversioned)
    public List<UProperty> ReadProperties(string type)
    {
        if (!DefinedStructures.Contains(type))
        {
            var schema = Mappings?.FindSchema(type);
            if (schema is null)
                throw new KeyNotFoundException($"Cannot find schema with name '{type}'");
		    
            DefinedStructures.Add(new UStruct(schema, Mappings!));
        }

        return ReadProperties(DefinedStructures[type] ?? throw new KeyNotFoundException("How'd we get here?"));
    }
    
    // TODO eventually redo when I add pak assets (not unversioned)
    public abstract List<UProperty> ReadProperties(UStruct structure);
    public abstract void WriteProperties(Writer writer, string type, List<UProperty> properties);

    /// <summary>
    /// Serializes the entire asset to a stream.
    /// </summary>
    /// <param name="writer"></param>
    public abstract void WriteAll(Writer writer);
    public abstract void WriteHeader(Writer writer);

    public int ReferenceOrAddString(string str)
    {
        if (!NameMap.Contains(str))
            NameMap.Add(str);

        return NameMap.GetIndex(str);
    }

    public void PostLoad()
    {
        foreach (var obj in ExportObjectsToLoad)
            obj.Load();
    }

    public abstract ResolvedObject? ResolvePackageIndex(FPackageIndex? index);

    public struct Export
    {
        public string Class;
        public string Name;
        public List<Property> Properties;
    }

    public struct Property
    {
        public string Type;
        public string Name;
        public object? Value;
    }

    public struct DictionaryPropertyEntry
    {
        public object Key;
        public object Value;
    }
    
    public override string ToString()
    {
        var exportArr = new List<Export>();
        foreach (var export in Exports)
        {
            var obj = new Export
            {
                Class = export.Class?.Name ?? "None",
                Name = export.Name,
                Properties = []
            };

            foreach (var prop in export.Properties)
            {
                var serialized = SerializeProperty(prop);
                obj.Properties.Add(serialized);
            }
            
            exportArr.Add(obj);
        }

        return JsonConvert.SerializeObject(exportArr, Formatting.Indented);

        Property SerializeProperty(UProperty property)
        {
            var type = property.Data?.Type;
            
            var result = new Property
            {
                Type = property.Data?.Type ?? "None",
                Name = property.Name,
                Value = new object()
            };

            if (property.Value is null)
            {
                Warning($"Value for '{property.Name}' was null, skipping serialization.");
                return result;
            }

            switch (type)
            {
                case null:
                    Error($"'{property.Name}' had a null type. Unable to serialize.");
                    return result;
                case "ArrayProperty":
                {
                    var values = property.Value?.As<ArrayProperty>();
                    var objects = new List<object>();

                    for (var i = 0; i < values!.Value!.Count; i++)
                    {
                        var value = values.Value![i];
                        var prop = (AbstractProperty)value;
                        var serialized = SerializeProperty(new UProperty
                        {
                            Name = prop.Name ?? i.ToString(),
                            Data = new PropertyData { Type = prop.GetPropertyType() },
                            Value = prop
                        });
                        
                        objects.Add(serialized);
                    }

                    result.Value = objects;
                    break;
                }
                case "MapProperty":
                {
                    var values = property.Value?.As<MapProperty>();
                    var objects = new List<DictionaryPropertyEntry>();

                    foreach (var kvp in values!.Value!)
                    {
                        var kvpKey = (AbstractProperty)kvp.Key;
                        var key = SerializeProperty(new UProperty
                        {
                            Name = kvpKey.Name ?? "MapEntry",
                            Data = new PropertyData { Type = kvpKey.GetPropertyType() },
                            Value = kvpKey
                        });
                        
                        var kvpValue = (AbstractProperty)kvp.Value;
                        var value = SerializeProperty(new UProperty
                        {
                            Name = kvpValue.Name ?? "MapEntry",
                            Data = new PropertyData { Type = kvpValue.GetPropertyType() },
                            Value = kvpValue
                        });

                        objects.Add(new DictionaryPropertyEntry
                        {
                            Key = key,
                            Value = value
                        });
                    }

                    result.Value = objects;
                    break;
                }
                case "StructProperty":
                {
                    var value = property.Value?.As<StructProperty>();
                    var objects = new List<Property>();
                    var holder = value?.Holder;

                    if (value?.Value is FInstancedStruct struc)
                    {
                        var props = struc.Properties;
                        objects.AddRange(props.Select(SerializeProperty));
                    }
                    else if (holder is null)
                    {
                        var obj = value?.Value;
                        GatherFields(obj, objects);
                    }
                    else
                    {
                        foreach (var uProp in holder.Properties)
                        {
                            var prop = new Property
                            {
                                Type = uProp.Data?.Type ?? "None",
                                Name = uProp.Name,
                                Value = SerializeProperty(uProp).Value
                            };
                            
                            objects.Add(prop);
                        }
                    }

                    result.Value = objects;
                    break;
                }
                default:
                {
                    var value = property.Value.As<AbstractProperty>().ValueAsObject;

                    if (value is IUnrealType)
                    {
                        var properties = new List<Property>();
                        GatherFields(value, properties);

                        result.Value = properties;
                    }
                    else
                    {
                        result.Value = value;
                    }

                    break;
                }
            }

            void GatherFields(object? obj, List<Property> properties)
            {
                if (obj is null)
                    return;
                
                var structType = obj.GetType();
                var fields = structType.GetFields();

                foreach (var field in fields)
                {
                    if (field.DeclaringType != structType) // I don't know why I can't figure out how to exclude derived fields
                        continue;
                    
                    if (field.GetCustomAttribute<UField>() == null)
                        continue;
                                
                    var prop = new Property
                    {
                        Type = field.FieldType.Name,
                        Name = field.Name,
                        Value = field.GetValue(obj)
                    };
                            
                    properties.Add(prop);
                }

                var methods = structType.GetMethods();

                foreach (var method in methods)
                {
                    if (method.GetCustomAttribute<UValueGetter>() == null)
                        continue;

                    var value = method.Invoke(obj, []);
                    var valueType = value!.GetType();
                    
                    properties.Add(new Property
                    {
                        Type = valueType.Name,
                        Name = "Value",
                        Value = value
                    });
                }
            }

            return result;
        }
    }
}