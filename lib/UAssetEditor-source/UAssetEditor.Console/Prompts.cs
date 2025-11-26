using Spectre.Console;

namespace UAssetEditor.Console;

public static class Prompts
{
    public static string Ask(string markup)
    {
        AnsiConsole.MarkupLine(markup);
        return AnsiConsole.Prompt(new TextPrompt<string>("[white]>>>[/]"));
    }
}