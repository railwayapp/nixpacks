package Nixpacks::Util::Logger;

sub new {
    my ($class, $tag) = @_;
    my $self = bless { tag => $tag }, $class;
}

sub log {
    my ($self, $color, $message_type, $message) = @_;
    my $tag = $self->{tag};
    CORE::say "\e[${color}m[$tag:$message_type]\e[0m $message";
}

sub info {
    my ($self, $message) = @_;
    $self->log(34, "info", $message);
}

sub warn {
    my ($self, $message) = @_;
    $self->log(33, "warn", $message);
}

sub err {
    my ($self, $message) = @_;
    $self->log(31, "error", $message);
}

1;