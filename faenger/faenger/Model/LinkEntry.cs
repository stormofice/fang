namespace faenger.Model;

public record LinkEntry(int Id, string Title, string Url, DateTime TimeCreated)
{
    // Note: The 0 is treated as the default value for Id, so EF auto-generates it (ick)
    public static LinkEntry Create(CreateLinkRequest link)
        => new(0, link.Title, link.Url, DateTime.UtcNow);
}
