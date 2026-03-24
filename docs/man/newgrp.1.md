# newgrp(1) - log in to a new group

## NAME

newgrp - log in to a new group

## SYNOPSIS

**newgrp** [*group*]

## DESCRIPTION

The **newgrp** command is used to change the current group ID during a
login session. If the optional *group* argument is given, the effective
group ID is changed to that group; otherwise the effective group ID is
changed to the user's primary group from /etc/passwd.

If the user is not a member of the specified group, and the group has a
password set in /etc/gshadow, the user will be prompted for the group
password. Root always has access to any group without a password prompt.

A new shell is started with the changed group ID. The shell is
determined by the **SHELL** environment variable, falling back to
/bin/sh.

## OPTIONS

None. Only an optional positional group name argument is accepted.

## EXIT STATUS

**0**
:   Success (though note that **newgrp** replaces the current process
    with a new shell via **execv**(2), so exit status 0 is not normally
    returned to the caller).

**1**
:   Permission denied, group not found, or unable to execute shell.

## FILES

/etc/group
:   Group account information.

/etc/gshadow
:   Secure group account information (for group passwords).

## SEE ALSO

groups(1), id(1), login(1), sg(1), group(5), gshadow(5)
