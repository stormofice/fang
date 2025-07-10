using faenger.Model;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddDatabaseDeveloperPageExceptionFilter();

builder.Services.AddDbContext<LinkContext>(options => options.UseSqlite());

var app = builder.Build();

app.UseHttpsRedirection();

if (app.Environment.IsDevelopment())
{
    app.UseDeveloperExceptionPage();
    app.UseMigrationsEndPoint();
}

using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<LinkContext>();
    db.Database.EnsureCreated();
}

app.MapGet("/", () => "Faenger API is here ^-^");

app.MapGet("/links", async (LinkContext db) => await db.Links.ToListAsync());
app.MapPost("/links", async (LinkContext db, CreateLinkRequest request) =>
{
    var link = LinkEntry.Create(request);
    db.Links.Add(link);
    await db.SaveChangesAsync();
    return TypedResults.Created($"/links/{link.Id}", link);
});

app.Run();
