# Artemis

The FCC publishes weekly full dumps and daily updates to their license database. The format is awful and the FCC search page is so slow. This dumps it into SQL so we can be speedy

## Usage
Before running the program, you must create your database. Run `sqlite3 fcc.db < migrations/01-create-db.sql` to do so.

Once this is done, you can run artemis. Note that there's no checking of if a key exists yet or not. It'll simply yolo it, so if you run it multiple times you'll get multiple copies. This will be fixed Eventually.

## References
All documentation for the FCC ULS database dumps can be found at [www.fcc.gov/wireless/data/public-access-files-database-downloads](https://www.fcc.gov/wireless/data/public-access-files-database-downloads)

Some very general information about the database can be found at [www.fcc.gov/sites/default/files/pubacc_intro_11122014.pdf](https://www.fcc.gov/sites/default/files/pubacc_intro_11122014.pdf). It's not very helpful though.

[www.fcc.gov/sites/default/files/pubacc_tbl_abbr_names_08212007.pdf](https://www.fcc.gov/sites/default/files/pubacc_tbl_abbr_names_08212007.pdf) contains some information about table and files names.

[www.fcc.gov/sites/default/files/public_access_database_definitions_v8.pdf](https://www.fcc.gov/sites/default/files/public_access_database_definitions_v8.pdf) will likely be the most useful reference. The file name will be the same as the value of the first column, which will also be the record type. You can use this to find the rest of the data on the entry type.

**NOTE:** In theory, this structure can also be algorithmically determined from [www.fcc.gov/sites/default/files/public_access_database_definitions_sql_v4.txt](https://www.fcc.gov/sites/default/files/public_access_database_definitions_sql_v4.txt). However, that is a can of worms I don't want to unpack anytime soon.

To start out, you will need the weekly dump, as daily dumps only contain changes. The latest weekly file can be found at [data.fcc.gov/download/pub/uls/complete/l_amat.zip](https://data.fcc.gov/download/pub/uls/complete/l_amat.zip)

### Special Conditions
The Special Conditions Code are defined in https://www.fcc.gov/file/20669/download. It is another pipe-separated value file, but this one has no header whatsoever. My reverse engineering of it is below. The only codes that show up in the amateur radio dumps are 999 and 4020 but I wanna be complete.

The format seems to be vaguely as follows:
```csv
code|service?|text1|text2|text3|text4|text5|[RPT]?
```
