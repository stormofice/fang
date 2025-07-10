using Microsoft.EntityFrameworkCore;

namespace faenger.Model;

public class LinkContext: DbContext
{
    public DbSet<LinkEntry> Links { get; set; }

    private string DbPath { get; }

    public LinkContext(DbContextOptions<LinkContext> options): base(options)
    {
        var path = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
        DbPath = Path.Join(path, "faenger.db");
    }

    protected override void OnConfiguring(DbContextOptionsBuilder options)
        => options.UseSqlite($"Data Source={DbPath}");
}
