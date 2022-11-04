# Artemis

The FCC publishes weekly full dumps and daily updates to their license database. The format is awful and the FCC search page is so slow. This dumps it into SQL so we can be speedy

## References
All documentation for the FCC ULS database dumps can be found at [www.fcc.gov/wireless/data/public-access-files-database-downloads](https://www.fcc.gov/wireless/data/public-access-files-database-downloads)

Some very general information about the database can be found at [www.fcc.gov/sites/default/files/pubacc_intro_11122014.pdf](https://www.fcc.gov/sites/default/files/pubacc_intro_11122014.pdf). It's not very helpful though.

[www.fcc.gov/sites/default/files/pubacc_tbl_abbr_names_08212007.pdf](https://www.fcc.gov/sites/default/files/pubacc_tbl_abbr_names_08212007.pdf) contains some information about table and files names.

[www.fcc.gov/sites/default/files/public_access_database_definitions_v8.pdf](https://www.fcc.gov/sites/default/files/public_access_database_definitions_v8.pdf) will likely be the most useful reference. The file name will be the same as the value of the first column, which will also be the record type. You can use this to find the rest of the data on the entry type.
