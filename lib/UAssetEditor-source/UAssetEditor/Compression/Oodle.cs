using System.Data;

namespace UAssetEditor.Compression;

public static class Oodle
{
    private static OodleDotNet.Oodle? _library;

    public static void Initialize(string dllPath)
    {
        _library = new OodleDotNet.Oodle(dllPath);
    }

    public static byte[] Decompress(byte[] data, int uncompressedSize)
    {
        if (_library is null)
            throw new NoNullAllowedException("Oodle library must be initialized!");
        
        var result = new byte[uncompressedSize];
        _library.Decompress(data, 0, data.Length, result, 0, result.Length);

        return result;
    }
} 