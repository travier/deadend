# FCOS Dead End MOTD writer

This is an experiment to write a simple DBus daemon in Rust.

Currently only working with the session bus:

```
$ cargo run --release -- --user
[INFO  deadend] Received: Signal NameAcquired from org.freedesktop.DBus
[INFO  deadend] Received: Signal NameAcquired from org.freedesktop.DBus
[INFO  deadend] Writing MOTD with reason: Test
[INFO  deadend] Successfully wrote MOTD
```

```
$ busctl --user call org.coreos.FcosDeadEnd /org/coreos/FcosDeadEnd org.coreos.FcosDeadEnd1 WriteDeadendReason s "Sample reason"
```
