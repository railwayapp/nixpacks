const std = @import("std");

pub fn main() anyerror!void {
    const stdout = std.io.getStdOut().writer();
    nosuspend stdout.print("Hello from Zig\n", .{}) catch return;
}

test "basic test" {
    try std.testing.expectEqual(10, 3 + 7);
}
