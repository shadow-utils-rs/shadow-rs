# chpasswd(8) - update passwords in batch mode

## NAME

chpasswd - update passwords in batch mode

## SYNOPSIS

**chpasswd** [*options*]

## DESCRIPTION

The **chpasswd** command reads a list of username:password pairs from
standard input and uses this information to update a group of existing
users. Each line is of the format:

    username:password

By default the password is expected to be in cleartext (not yet
supported in shadow-rs; use **-e**). With the **-e** flag, the password
is expected to be already encrypted (pre-hashed).

## OPTIONS

**-c**, **--crypt-method** *METHOD*
:   Use the specified crypt method. Supported values: SHA256, SHA512,
    YESCRYPT, DES, MD5.

**-e**, **--encrypted**
:   Supplied passwords are already encrypted (pre-hashed).

**-m**, **--md5**
:   Use MD5 encryption for cleartext passwords (deprecated).

**-R**, **--root** *CHROOT_DIR*
:   Apply changes in the *CHROOT_DIR* directory.

**-s**, **--sha-rounds** *ROUNDS*
:   Use the specified number of rounds for SHA256/SHA512 encryption.

## EXIT STATUS

**0**
:   Success.

**1**
:   Permission denied, invalid input, file busy, or unexpected failure.

## FILES

/etc/shadow
:   Secure user account information.

## SEE ALSO

passwd(1), passwd(5), shadow(5)
