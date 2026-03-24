# chsh(1) - change login shell

## NAME

chsh - change login shell

## SYNOPSIS

**chsh** [*options*] [*LOGIN*]

## DESCRIPTION

The **chsh** command changes the user login shell. This determines the
name of the user's initial login command. A normal user may only change
the login shell for their own account; the superuser may change the login
shell for any account.

The new shell must be listed in /etc/shells unless the caller is root.

## OPTIONS

**-l**, **--list-shells**
:   Print the list of shells listed in /etc/shells and exit.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-s**, **--shell** *SHELL*
:   Set the login shell to *SHELL*. The shell must be an absolute path
    and must be listed in /etc/shells (unless the caller is root).

## EXIT STATUS

**0**
:   Success.

**1**
:   Permission denied or operation failed.

## FILES

/etc/passwd
:   User account information.

/etc/shells
:   List of valid login shells.

## SEE ALSO

chfn(1), login(1), passwd(5), shells(5)
