# Bookmarks

This is a utility to do things with plaintext bookmarks.

I store my bookmarks in something like...

    title: TITLE
    url: https://www.website.com
    date: 2020-02-05

    @tag1 @tag2

With this util, you can do:

    bookmarks new|find|open|view ARGS...

- `new`: prompt to create a new bookmark from the commandline
- `find`: list all bookmarks with tags matching ARGS (args given without '@')
- `open`: prompt to open a bookmark (using xdg-open) matching ARGS
- `view`: prompt to print a bookmark's file contents to screen (matching ARGS)
