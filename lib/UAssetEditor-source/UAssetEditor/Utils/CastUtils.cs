namespace UAssetEditor.Utils;

public static class CastUtils
{
    public static T As<T>(this object obj) => (T)obj;
    public static T? AsOrDefault<T>(this object obj) where T : class => obj as T;
}