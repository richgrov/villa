const std = @import("std");

const types = @cImport(@cInclude("protocol/types.h"));

test "protocol short" {
    const short_buf = [_]u8{ 0x13, 0x07 };
    try std.testing.expectEqual(4871, types.read_mc_short(&short_buf));
}

test "protocol int" {
    const int_buf = [_]u8{ 0x44, 0xE1, 0x11, 0xA7 };
    try std.testing.expectEqual(1155600807, types.read_mc_int(&int_buf));

    var write_buf: [4]u8 = undefined;
    _ = types.write_mc_int(&write_buf, 1155600807);
    try std.testing.expectEqual(int_buf, write_buf);
}

test "protocol long" {
    const long_buf = [_]u8{ 0xFF, 0x10, 0x7C, 0x99, 0x00, 0x65, 0x9A, 0x0D };
    try std.testing.expectEqual(-67416997832058355, types.read_mc_long(&long_buf));

    var write_buf: [8]u8 = undefined;
    _ = types.write_mc_long(&write_buf, -67416997832058355);
    try std.testing.expectEqual(long_buf, write_buf);
}
