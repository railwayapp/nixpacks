# https://stackoverflow.com/a/3738367
package Nixpacks::Util::ChmodRecursive;

use File::Find;

sub chmod_recursive {
    my ($dir) = @_;
    sub wanted
    {
        my $perm = -d $File::Find::name ? 0555 : 0444;
        chmod $perm, $File::Find::name;
    }
    find(\&wanted, $dir);
}

1;