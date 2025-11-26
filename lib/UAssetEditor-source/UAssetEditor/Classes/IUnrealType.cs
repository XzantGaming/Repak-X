namespace UAssetEditor.Classes;

[AttributeUsage(AttributeTargets.Field)]
public class UField : Attribute
{
    
}

// Ik this is so scuffed but whatever - owen
[AttributeUsage(AttributeTargets.Method)]
public class UValueGetter : Attribute
{
    
}

public interface IUnrealType
{
    
}