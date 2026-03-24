# pwck(8) - verify integrity of password files

## NAME

pwck - verify integrity of password files

## SYNOPSIS

**pwck** [*options*] [*passwd* [*shadow*]]

## DESCRIPTION

The **pwck** command verifies the integrity of the system authentication
information. It checks that all entries in /etc/passwd and /etc/shadow
have the proper format and contain valid data.

Checks performed include:

- Correct number of fields
- Unique and valid user names
- Valid user and group identifiers
- Valid primary group
- Valid home directory
- Valid login shell
- Matching passwd/shadow entries

## OPTIONS

**-q**, **--quiet**
:   Report only errors, suppress warnings.

**-r**, **--read-only**
:   Display errors and warnings but do not modify files.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-s**, **--sort**
:   Sort entries by UID.

## EXIT STATUS

**0**
:   Success.

**1**
:   Invalid command syntax.

**2**
:   One or more bad password entries.

**3**
:   Cannot open files.

**4**
:   Cannot lock files.

**5**
:   Cannot update files.

## FILES

/etc/passwd
:   User account information.

/etc/shadow
:   Secure user account information.

/etc/group
:   Group account information.

## SEE ALSO

passwd(5), shadow(5), grpck(8)
