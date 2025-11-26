using System.Dynamic;
using UAssetEditor.Unreal.Properties;
using UAssetEditor.Unreal.Properties.Types;

namespace UAssetEditor.Utils;

public static class JsonUtils
{
    private struct PropertyObject
    {
        public string Type;
        public dynamic? Value;
    }
    
    private static object PropertyToObject(AbstractProperty? property)
    {
        var obj = new PropertyObject { Type = property?.GetType().Name ?? "None" };
        dynamic expando = new ExpandoObject();
        
        switch (property)
        {
            case null:
                return obj;
            case ArrayProperty array:
            {
                var arrayObj = new List<object>();
                foreach (var elm in array.Value ?? new())
                {
                    var value = PropertyToObject((AbstractProperty)elm);
                    arrayObj.Add(value);
                }

                ((IDictionary<string, object>)expando)[array.Name ?? "Unnamed Array"] = arrayObj;

                break;
            }
            case MapProperty map:
            {
                var dict = new Dictionary<object, object>();
                foreach (var kvp in map.Value ?? new())
                {
                    var key = PropertyToObject((AbstractProperty)kvp.Key);
                    var value = PropertyToObject((AbstractProperty)kvp.Value);
                    dict.Add(key, value);
                }
                
                ((IDictionary<string, object>)expando)[map.Name ?? "Unnamed Map"] = dict;

                break;
            }
            case StructProperty struc:
            {
                
                if (struc.Value is List<UProperty> props)
                {
                    
                    foreach (var kvp in props)
                    {
                        var key = kvp.Name;
                        var value = PropertyToObject((AbstractProperty?)kvp.Value);
                        ((IDictionary<string, object>)expando)[key] = value;
                    }
                }
                else
                {
                    obj.Value = struc.Value;
                }

                break;
            }
            default:
                ((IDictionary<string, object?>)expando)[property.Name ?? "Value"] = property.ValueAsObject;
                break;
        }

        obj.Value = expando;
        return obj;
    }
    
    // TODO fix
    /*public static string ToJsonString(this Asset asset)
    {
        var objs = new List<object>();
        
        foreach (var export in asset.Exports)
        {
            var properties = new List<object>();

            foreach (var prop in export.Properties)
            {
                var propObj = PropertyToObject((AbstractProperty?)prop.Value);
                properties.Add(propObj);
            }
            
            var obj = new
            {
                Type = export.Class?.Name,
                Name = export.Name,
                Properties = properties
            };
            
            objs.Add(obj);
        }
        
        return JsonConvert.SerializeObject(objs, Formatting.Indented);
    }*/
}