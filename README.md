# Backup Applicator
A simple rust based program for installing world backups 
created by various Minecraft Mods on a Dedicated server.

```
USAGE:
    backup-applicator [OPTIONS] --instance <FOLDER> --backup <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --backup <FILE>           The backup to apply.
    -i, --instance <FOLDER>       The instance folder to apply the backup to.
    -v, --verbose <verbose>...    Enables verbose logging, can be provided multiple times for
                                  increasingly verbose logging.
```
Currently only zip based backups are supported, Backup applicator shouldn't 
care where your backup is located inside the zip as it will locate your `level.dat` file
inside the backup and install it appropriately.  
Backup Applicator will also load your `server.properties` file from the specified instance folder, 
in order to use the appropriate `Level-Name` property, if it is not found, Backup Applicator will fall
back to `world`.
