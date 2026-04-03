# usermod(8) - modify a user account

## NAME

usermod - modify a user account

## SYNOPSIS

**usermod** [*options*] *LOGIN*

## DESCRIPTION

The **usermod** command modifies the system account files to reflect the
changes that are specified on the command line.

## OPTIONS

**-a**, **--append**
:   Append the user to the supplementary group(s) specified by **-G**.
    Use only with the **-G** option.

**-c**, **--comment** *COMMENT*
:   Set the new value of the user's GECOS field.

**-d**, **--home** *HOME_DIR*
:   Set the new home directory for the user.

**-e**, **--expiredate** *EXPIRE_DATE*
:   Set the account expiration date.

**-f**, **--inactive** *INACTIVE*
:   Set the password inactive period.

**-g**, **--gid** *GROUP*
:   Set the new primary group ID (numeric).

**-G**, **--groups** *GROUPS*
:   Set the list of supplementary groups (comma-separated). If the **-a**
    option is not used, the user is removed from all groups not listed.

**-l**, **--login** *NEW_LOGIN*
:   Change the user's login name.

**-L**, **--lock**
:   Lock the user's password by prepending a '!' to the shadow password.

**-p**, **--password** *PASSWORD*
:   Set the user's password to the specified pre-hashed value. The hash
    must not contain ':', '\\n', or '\\r' characters.

**-P**, **--prefix** *PREFIX_DIR*
:   Use *PREFIX_DIR* as a prefix for system file paths.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-s**, **--shell** *SHELL*
:   Set the new login shell.

**-u**, **--uid** *UID*
:   Set the new numeric user ID.

**-U**, **--unlock**
:   Unlock the user's password by removing the '!' prefix from the
    shadow password.

## EXIT STATUS

**0**
:   Success.

**1**
:   Cannot update password file.

**2**
:   Invalid command syntax.

**4**
:   UID already in use.

**6**
:   User does not exist.

## FILES

/etc/passwd
:   User account information.

/etc/shadow
:   Secure user account information.

/etc/group
:   Group account information.

## SEE ALSO

useradd(8), userdel(8), groupmod(8), passwd(1)
