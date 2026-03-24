# useradd(8) - create a new user

## NAME

useradd - create a new user or update default new user information

## SYNOPSIS

**useradd** [*options*] *LOGIN*
**useradd** **-D** [*options*]

## DESCRIPTION

The **useradd** command creates a new user account using the values
specified on the command line plus the default values from the system.
The new user account will be entered into the system files as needed,
the home directory will be created, and initial files copied, depending
on the command line options.

When invoked with the **-D** flag, **useradd** displays or updates the
default values used for creating new accounts.

## OPTIONS

**-c**, **--comment** *COMMENT*
:   Set the GECOS field of the new account.

**-d**, **--home-dir** *HOME_DIR*
:   Set the home directory of the new account.

**-D**, **--defaults**
:   Print or change default useradd configuration.

**-e**, **--expiredate** *EXPIRE_DATE*
:   Set the expiration date of the new account (YYYY-MM-DD).

**-f**, **--inactive** *INACTIVE*
:   Set the password inactivity period of the new account.

**-g**, **--gid** *GROUP*
:   Set the name or numeric ID of the primary group of the new account.

**-G**, **--groups** *GROUPS*
:   Set the list of supplementary groups of the new account (comma-separated).

**-k**, **--skel** *SKEL_DIR*
:   Specify the skeleton directory (default: /etc/skel).

**-m**, **--create-home**
:   Create the user's home directory if it does not exist.

**-M**, **--no-create-home**
:   Do not create the user's home directory.

**-N**, **--no-user-group**
:   Do not create a group with the same name as the user.

**-o**, **--non-unique**
:   Allow creating users with duplicate (non-unique) UIDs. Requires **-u**.

**-p**, **--password** *PASSWORD*
:   Set the encrypted password of the new account.

**-r**, **--system**
:   Create a system account.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-s**, **--shell** *SHELL*
:   Set the login shell of the new account.

**-u**, **--uid** *UID*
:   Set the user ID of the new account.

**-U**, **--user-group**
:   Create a group with the same name as the user (default behavior).

## EXIT STATUS

**0**
:   Success.

**1**
:   Cannot update password file.

**2**
:   Invalid command syntax.

**3**
:   Invalid argument to option.

**4**
:   UID already in use (and no **-o**).

**6**
:   Specified group does not exist.

**9**
:   Username already in use.

**10**
:   Cannot update group file.

**12**
:   Cannot create home directory.

**14**
:   Cannot update SELinux user mapping.

## FILES

/etc/passwd
:   User account information.

/etc/shadow
:   Secure user account information.

/etc/group
:   Group account information.

/etc/gshadow
:   Secure group account information.

/etc/login.defs
:   Shadow password suite configuration.

/etc/default/useradd
:   Default values for account creation.

/etc/skel
:   Directory containing default files.

## SEE ALSO

userdel(8), usermod(8), groupadd(8), login.defs(5)
