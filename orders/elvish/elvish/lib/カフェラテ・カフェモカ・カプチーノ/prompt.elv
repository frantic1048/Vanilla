use platform
use str

var ssh-prefix = ''
if (or (has-env SSH_CONNECTION) (has-env SSH_CLIENT) (has-env SSH_TTY)) {
    set ssh-prefix = '[ssh '(platform:hostname)'] '
}

# Keep short paths whole; otherwise retain the last two directories.
fn prompt-path {|directory|
    var display = (tilde-abbr $directory)
    if (> (count [(str:split '' $display)]) 40) {
        var parts = [(str:split / $display)]
        if (> (count $parts) 3) {
            set display = $parts[0]'/.../'(str:join / $parts[-2..])
        }
    }
    put $display
}

fn styled-prompt-path {|directory|
    var display = (prompt-path $directory)
    var parts = [(str:split / $display)]
    var parent = (str:join / $parts[..-1])
    if (> (count $parent) 0) {
        put (styled $parent'/' bright-blue) (styled $parts[-1] bold bright-blue)
    } else {
        put (styled $display bright-blue)
    }
}

var shell-indicator = (styled 'elv❯' bold cyan)
set edit:prompt = {
    put $ssh-prefix (styled-prompt-path $pwd) "\n" $shell-indicator ' '
}

# keep blank, it is not resizing friendly :(
set edit:rprompt = { put '' }
