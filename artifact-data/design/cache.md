# SPC-data-db
> This specification only exists to improve performance

Significant speed gains can be made by caching artifact data into a local
SQLite database in `.art/db.sql`. For linux specifically (and probably MacOS,
don't know about windows) you can get the timestamp when any file was modified.

- If we only parsed files where the timestamp changed we could speed things up
  significantly for re-finding the implementation references.
