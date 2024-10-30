pub const VERSION: &str = "1.0.0";
pub const PROGRAM_NAME: &str = "adbr";

pub const DEFAULT_ADB_SERVER_IP: &str = "127.0.0.1";
pub const DEFAULT_ADB_SERVER_PORT: u16 = 5037;

pub const ADB_ADDRESS_ENV: &str = "ADB_ADDRESS";
pub const ADB_SERVER_CONNECT_TIMEOUT_SECONDS_DURATION: u64 = 5;

pub const FLAG_HELP_SHORT: &str = "-h";
pub const FLAG_HELP_LONG: &str = "--help";
pub const FLAG_VERSION: &str = "--version";
pub const FLAG_SERVER_ADDRESS: &str = "-H";
pub const FLAG_SERVER_PORT: &str = "-P";
pub const FLAG_SERIAL: &str = "-s";
pub const FLAG_USB: &str = "-d";
pub const FLAG_EMULATOR: &str = "-e";
pub const FLAG_WATCH_DEVICES: &str = "-w";
pub const FLAG_TIMEOUT: &str = "-t";

pub const ADB_SHELL_COMMAND: &str = "shell:";
pub const ADB_DEVICES_COMMAND: &str = "host:devices";
pub const HOST_FORWARD_COMMAND: &str = "host:forward";
pub const HOST_FORWARD_KILL_COMMAND: &str = "host:killforward";
pub const HOST_FORWARD_KILL_ALL_COMMAND: &str = "host:killforward-all";
pub const HOST_FORWARD_LIST_COMMAND: &str = "host:list-forward";

pub const HOST_REVERSE_COMMAND: &str = "reverse:forward";
pub const HOST_REVERSE_REMOVE_COMMAND: &str = "reverse:killforward";
pub const HOST_REVERSE_REMOVE_ALL_COMMAND: &str = "reverse:killforward-all";
pub const HOST_REVERSE_LIST_COMMAND: &str = "reverse:list-forward";


pub const HOST_SERIALNO_COMMAND: &str = "host:get-serialno";
pub const HOST_GET_DEVPATH_COMMAND: &str = "host:get-devpath";

pub const USER_ENABLE_VERITY_COMMAND: &str = "enable-verity";
pub const USER_KEYGEN_COMMAND: &str = "keygen";
pub const USER_DISABLE_VERITY_COMMAND: &str = "disable-verity";
pub const USER_BUGREPORT_COMMAND: &str = "bugreport";
pub const USER_LOGCAT_COMMAND: &str = "logcat";
pub const USER_TRANSPORT_COMMAND: &str = "transport";
pub const USER_REBOOT_COMMAND: &str = "reboot";
pub const USER_SERIALNO_COMMAND: &str = "get-serialno";
pub const USER_REMOUNT_COMMAND: &str = "remount";
pub const USER_SHELL_COMMAND: &str = "shell";
pub const USER_DEVICES_COMMAND: &str = "devices";
pub const USER_CONNECT_COMMAND: &str = "connect";
pub const USER_FORWARD_COMMAND: &str = "forward";
pub const USER_REVERSE_COMMAND: &str = "reverse";
pub const USER_PUSH_COMMAND: &str = "push";
pub const USER_PULL_COMMAND: &str = "pull";
pub const USER_EXIT_COMMAND: &str = "exit\n";
pub const USER_USB_COMMAND: &str = "usb";
pub const USER_TCPIP_COMMAND: &str = "tcpip";
pub const USER_WAIT_FOR_COMMAND: &str = "wait-for";
pub const USER_GET_STATE_COMMAND: &str = "get-state";

pub const OPTION_NO_REBIND: &str = "--no-rebind";
pub const OPTION_REMOVE: &str = "--remove";
pub const OPTION_REMOVE_ALL: &str = "--remove-all";
pub const OPTION_LIST: &str = "--list";

pub const USER_GET_DEVPATH_COMMAND: &str = "get-devpath";
pub const NO_REBIND_OPTION: &str = "norebind";
pub const SYNC_COMMAND: &str = "sync:";

pub const REBOOT_BOOTLOADER: &str = "bootloader";
pub const REBOOT_RECOVERY: &str = "recovery";
pub const REBOOT_SIDELOAD: &str = "sideload";
pub const REBOOT_SIDELOAD_AUTO_REBOOT: &str = "sideload-auto-reboot";

pub const ADB_REBOOT_COMMAND: &str = "reboot:";
pub const ADB_REMOUNT_COMMAND: &str = "remount:";
pub const ADB_ROOT_COMMAND: &str = "root:";
pub const ADB_UNROOT_COMMAND: &str = "unroot:";
pub const ADB_USB_COMMAND: &str = "usb:";
pub const ADB_TCPIP_COMMAND: &str = "tcpip:";
pub const ADB_REBOOT_BOOTLOADER_COMMAND: &str = "reboot:bootloader";
pub const ADB_REBOOT_RECOVERY_COMMAND: &str = "reboot:recovery";
pub const ADB_REBOOT_SIDELOAD_COMMAND: &str = "reboot:sideload";
pub const ADB_REBOOT_SIDELOAD_AUTO_REBOOT_COMMAND: &str = "reboot:sideload-auto-reboot";
pub const ADB_GET_STATE_COMMAND: &str = "host:get-state";
pub const SEND_COMMAND: &str = "SEND";
pub const STAT_COMMAND: &str = "STAT";
pub const DATA_COMMAND: &str = "DATA";
pub const DONE_COMMAND: &str = "DONE";
pub const QUIT_COMMAND: &str = "QUIT";
pub const RECV_COMMAND: &str = "RECV";
pub const LIST_COMMAND: &str = "LIST";
pub const DENT_COMMAND: &str = "DENT";

pub const USER_INSTALL_COMMAND: &str = "install";
pub const PM_INSTALL: &str = "pm install";
pub const PM_UNINSTALL: &str = "pm uninstall";
pub const USER_UNINSTALL_COMMAND: &str = "uninstall";


pub const UNINSTALL_FLAG_KEEP_DATA: &str = "-k";
pub const INSTALL_FLAG_REPLACE: &str = "-r";
pub const INSTALL_FLAG_DOWNGRADE: &str = "-d";
pub const INSTALL_FLAG_GRANT_PERMISSIONS: &str = "-g";
pub const INSTALL_FLAG_TEST: &str = "-t";
pub const INSTALL_FLAG_SDCARD: &str = "-s";
pub const INSTALL_FLAG_INTERNAL: &str = "-f";
pub const INSTALL_FLAG_FORWARD_LOCK: &str = "-l";

pub const USER_ROOT_COMMAND: &str = "root";
pub const USER_UNROOT_COMMAND: &str = "unroot";

pub const DEFAULT_WAIT_STATE: &str = "device";

pub const S_IFDIR: u32 = 0x4000;
pub const DEFAULT_PUSH_MODE: u32 = 0o644;  // r
pub const STAT_DATA_SIZE: usize = 12;

pub const OKAY: &str = "OKAY";
pub const FAIL: &str = "FAIL";

pub const DEVICE_TEMP_DIRECTORY: &str = "/data/local/tmp/";

pub const REFRESH_INTERVAL_SECS: u64 = 1;

pub const SELECT_TIMEOUT_USEC: i64 = 100_000;