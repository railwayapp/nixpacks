const std = @import("std");
const zzz = @import("zzz");

pub fn main() anyerror!void {
    const stdout = std.io.getStdOut();
    _ = nosuspend stdout.write("Hello from Zig\n") catch return;

    var tree = zzz.ZTree(1, 100){};
    _ = try tree.appendText("hello: world");
    tree.show();
}

test "basic test" {
    try std.testing.expectEqual(10, 3 + 7);
}
