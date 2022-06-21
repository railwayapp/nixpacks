const std = @import("std");
const uri = @import("uri");

pub fn main() anyerror!void {
    const stdout = std.io.getStdOut();
    _ = nosuspend stdout.write("Hello from Zig\n") catch return;

    _ = nosuspend stdout.write("The URI scheme of GitHub is " ++ ((try uri.parse("https://github.com/railwayapp/nixpacks")).scheme orelse "") ++ ".\n") catch return;
}

test "basic test" {
    try std.testing.expectEqual(10, 3 + 7);
}
