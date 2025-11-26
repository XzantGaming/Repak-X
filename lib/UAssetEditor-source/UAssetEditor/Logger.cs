global using static UAssetEditor.Logger;
using Serilog;

namespace UAssetEditor;

public enum LogLevel : byte
{
    Error, 
    Warn,
    Info,
    Everything
}

public static class Logger
{
    public static void StartLogger()
    {
        Log.Logger =   new LoggerConfiguration()
            .WriteTo.Console()
            .CreateLogger();
    }

    private static bool IsLogLevel(LogLevel level)
    {
        return Globals.LogLevel >= level;
    }

    public static void Information(string message)
    {
        if (IsLogLevel(LogLevel.Info))
            Log.Information(message);
    }
    
    public static void Warning(string message)
    {
        if (IsLogLevel(LogLevel.Warn))
            Log.Warning(message);
    }
    
    public static void Error(string message)
    {
        if (IsLogLevel(LogLevel.Error))
            Log.Error(message);
    }
}