# userdel(8) - delete a user account

## NAME

userdel - delete a user account and related files

## SYNOPSIS

**userdel** [*options*] *LOGIN*

## DESCRIPTION

The **userdel** command modifies the system account files, deleting all
entries that refer to the user name *LOGIN*. The named user must exist.

The user's entry is removed from /etc/passwd and /etc/shadow. The user
is also removed from membership lists in /etc/group and /etc/gshadow.

## OPTIONS

**-f**, **--force**
:   Force removal of the account even if the user is still logged in.
    Also forces removal of the home directory and mail spool.

**-P**, **--prefix** *PREFIX_DIR*
:   Use *PREFIX_DIR* as a prefix for system file paths.

**-r**, **--remove**
:   Remove the user's home directory and mail spool.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

## EXIT STATUS

**0**
:   Success.

**1**
:   Cannot update password file.

**2**
:   Invalid command syntax.

**10**
:   Cannot update group file.

**12**
:   Cannot remove home directory.

## FILES

/etc/passwd
:   User account information.

/etc/shadow
:   Secure user account information.

/etc/group
:   Group account information.

/etc/gshadow
:   Secure group account information.

## SEE ALSO

useradd(8), usermod(8), groupdel(8)
