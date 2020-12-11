# duplicates
Delete duplicate files, check hashes

Licence goes like this - if you like it - use it, develop it, build something else on it, 
I don't care, but also before running code - check sources, as _I don't guarantee that this app will not delete your important files_. 
*You have been warned and it is your responsibility to use this app*. If you want to help - write some tests :)


This tool makes organizing photos sane again. Config goes like this:
### if file path contains one of those, this file will not be processed
ignore_paths = ["src", "target",".git"] 

### where to start. goes into subdirectories too
working_dir = "."

### if duplicates were found, if they are in location like one of below - they are more likely to be deleted. The earlier it is on the list - the bigger chance
### Always all except one files are deleted.
delete_score = ["download", "DCIM", "random","organizeme"]

### Action can be: 
#### T - write which files would be deleted but don't delete
#### S - as T but stop program after first duplicate found
#### D - as T except DO delete files
#### everything else - display some diagnostic stuff
action="T"

###Watchdog
## true - after scanning finishes, application should monitor filesystem changes and recalculate hashes for them
## false - after scanning finishes, quit
watchdog = false

# Enjoy !
