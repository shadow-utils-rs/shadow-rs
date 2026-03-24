# chfn(1) - change user finger information

## NAME

chfn - change real user name and information

## SYNOPSIS

**chfn** [*options*] [*LOGIN*]

## DESCRIPTION

The **chfn** command changes the user finger information stored in the
GECOS field of /etc/passwd. This information is typically displayed by
the **finger**(1) program and includes the user's full name, office room
number, and phone numbers.

A normal user may only change their own finger information; the superuser
may change the information for any user. Only the superuser may change
the "other" field.

At least one option flag (**-f**, **-r**, **-w**, **-h**, or **-o**) must
be specified.

## OPTIONS

**-f**, **--full-name** *FULL_NAME*
:   Change the user's full name.

**-h**, **--home-phone** *HOME_PHONE*
:   Change the user's home phone number.

**-o**, **--other** *OTHER*
:   Change the user's other GECOS information. Only root may set this field.

**-r**, **--room** *ROOM*
:   Change the user's room number.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-w**, **--work-phone** *WORK_PHONE*
:   Change the user's office phone number.

## EXIT STATUS

**0**
:   Success.

**1**
:   Permission denied or operation failed.

## FILES

/etc/passwd
:   User account information.

## SEE ALSO

chsh(1), finger(1), passwd(5)
