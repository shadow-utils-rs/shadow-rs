# groupadd(8) - create a new group

## NAME

groupadd - create a new group

## SYNOPSIS

**groupadd** [*options*] *GROUP*

## DESCRIPTION

The **groupadd** command creates a new group account using the values
specified on the command line plus the default values from the system.
The new group will be entered into the system files (/etc/group and
/etc/gshadow) as needed.

## OPTIONS

**-f**, **--force**
:   Exit successfully if the group already exists, and cancel **-g** if
    the GID is already used (a new GID will be allocated instead).

**-g**, **--gid** *GID*
:   Use *GID* for the new group.

**-K**, **--key** *KEY=VALUE*
:   Override /etc/login.defs defaults (GID_MIN, GID_MAX, SYS_GID_MIN,
    SYS_GID_MAX). Can be specified multiple times.

**-o**, **--non-unique**
:   Allow creating a group with a non-unique (duplicate) GID.

**-p**, **--password** *PASSWORD*
:   Set the encrypted password for the new group.

**-P**, **--prefix** *PREFIX_DIR*
:   Use *PREFIX_DIR* as a prefix for system file paths.

**-r**, **--system**
:   Create a system group (allocated from the system GID range defined
    in /etc/login.defs).

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
:   GID already in use (and no **-o** or **-f**).

**9**
:   Group name already in use.

**10**
:   Cannot update group file.

## FILES

/etc/group
:   Group account information.

/etc/gshadow
:   Secure group account information.

/etc/login.defs
:   Shadow password suite configuration.

## SEE ALSO

groupdel(8), groupmod(8), login.defs(5)
