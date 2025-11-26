namespace UAssetEditor.Utils;

public static class ArrayUtils
{
    public static T[] Range<T>(this T[] array, int start, int length)
    {
        var buffer = new T[length];
        Buffer.BlockCopy(array, start, buffer, 0, length);
        return buffer;
    }
}