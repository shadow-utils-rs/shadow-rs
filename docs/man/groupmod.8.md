# groupmod(8) - modify a group definition

## NAME

groupmod - modify a group definition

## SYNOPSIS

**groupmod** [*options*] *GROUP*

## DESCRIPTION

The **groupmod** command modifies the definition of the specified
*GROUP* by modifying the appropriate entries in the group and gshadow
databases.

## OPTIONS

**-g**, **--gid** *GID*
:   Change the group ID to *GID*.

**-n**, **--new-name** *NEW_GROUP*
:   Change the name of the group to *NEW_GROUP*.

**-o**, **--non-unique**
:   Allow using a non-unique (duplicate) GID when used with **-g**.

**-p**, **--password** *PASSWORD*
:   Change the group password to the encrypted *PASSWORD*.

**-P**, **--prefix** *PREFIX_DIR*
:   Use *PREFIX_DIR* as a prefix for system file paths.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

## EXIT STATUS

**0**
:   Success.

**2**
:   Invalid command syntax.

**3**
:   Invalid argument to option.

**4**
:   GID already in use.

**6**
:   Group does not exist.

**9**
:   Group name already in use.

**10**
:   Cannot update group file.

## FILES

/etc/group
:   Group account information.

/etc/gshadow
:   Secure group account information.

## SEE ALSO

groupadd(8), groupdel(8)
