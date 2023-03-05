package Nixpacks::Nix;

sub get_nix_path {
    my ($exe) = @_;
    
    my $path = `which $exe`;
    $path =~ s/\n//;
    my $storePath = `nix-store -q $path`;
    $storePath =~ s/\n//;
    return $storePath;
}

1;