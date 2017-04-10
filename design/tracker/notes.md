### dump database schema
pg_dump <dbname> -s -F c -f <dumpname>

### restore from dump:
pg_restore <dumpname> -d <dbname>
(must specify an existing database to import to)

important: db.dump must be in user `postgres` home directory (or whatever user
	accesses postgres)
	
To find where this is:
	`sudo -u postgres -i`
	`readlink -f <anyFileInThisDir>`
	This should give you the path to the current directory
	`exit` to go back to normal user
	`mv db.dump <pathGivenByReadlink/db.dump>`
	
switch back to user postgres again

pg_restore should now work as above
