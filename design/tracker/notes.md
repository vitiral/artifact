### dump database schema
`pg_dump <dbname> -s -F c -f <dumpname>`

### restore from dump:
`pg_restore <dumpname> -d <dbname>`
(must specify an existing database to import to)

important: db.dump must be in user `postgres` home directory (or whatever user accesses postgres)
	
To find where this is:

* Change to user for postgresql: `sudo -u postgres -i` (for me the user is `postgres`)
	
* Get actual file location: `readlink -f <anyFileInThisDir>`
	
	This should give you the path to the current directory REMEMBER THIS LOCATION
	
* `exit` to go back to normal user

* as normal user `mv db.dump <pathGivenByReadlink/db.dump>`
	
switch back to user postgres again

pg_restore should now work as above (`pg_restore <dumpname> -d <dbname>`)
