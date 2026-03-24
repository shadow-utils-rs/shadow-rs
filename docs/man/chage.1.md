# chage(1) - change user password expiry information

## NAME

chage - change user password expiry information

## SYNOPSIS

**chage** [*options*] *LOGIN*

## DESCRIPTION

The **chage** command changes the number of days between password changes
and the date of the last password change. This information is used by
the system to determine when a user must change their password.

## OPTIONS

**-d**, **--lastday** *LAST_DAY*
:   Set the date of the last password change. The date may be expressed
    as a date (YYYY-MM-DD) or as the number of days since January 1, 1970.
    A value of -1 removes the last-change date requirement.

**-E**, **--expiredate** *EXPIRE_DATE*
:   Set the account expiration date. The date may be expressed as a date
    (YYYY-MM-DD) or as the number of days since January 1, 1970.
    A value of -1 removes the expiration date.

**-I**, **--inactive** *INACTIVE*
:   Set the number of days of inactivity after a password has expired
    before the account is locked. A value of -1 removes the inactivity
    requirement.

**-l**, **--list**
:   Show account aging information.

**-m**, **--mindays** *MIN_DAYS*
:   Set the minimum number of days between password changes. A value
    of -1 removes the minimum days requirement.

**-M**, **--maxdays** *MAX_DAYS*
:   Set the maximum number of days during which a password is valid.
    A value of -1 removes the maximum days requirement.

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-W**, **--warndays** *WARN_DAYS*
:   Set the number of days of warning before a password change is
    required. A value of -1 removes the warning.

## EXIT STATUS

**0**
:   Success.

**1**
:   Permission denied.

**2**
:   Invalid command syntax.

## FILES

/etc/shadow
:   Secure user account information.

## SEE ALSO

passwd(1), passwd(5), shadow(5)
