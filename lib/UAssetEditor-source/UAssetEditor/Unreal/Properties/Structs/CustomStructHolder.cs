namespace UAssetEditor.Unreal.Properties.Structs;

public class CustomStructHolder
{
    public List<UProperty> Properties;

    public CustomStructHolder(List<UProperty> properties)
    {
        Properties = properties;
    }

    public UProperty? this[string name] => Properties.FirstOrDefault(x => x.Name == name);

    public T? GetPropertyValue<T>(string name) where T : class
    {
        return Properties.FirstOrDefault(x => x.Name == name)?.Value as T;
    }
}