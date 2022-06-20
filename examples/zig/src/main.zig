const std = @import("std");

pub fn main() anyerror!void {
    const stdout = std.io.getStdOut().writer();
    nosuspend stdout.print("All your codebase are belong to us.\n", .{}) catch return;
}

test "basic test" {
    try std.testing.expectEqual(10, 3 + 7);
}
