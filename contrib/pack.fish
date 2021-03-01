function __fish_using_command
    set cmd (commandline -opc)
    if [ (count $cmd) -eq (count $argv) ]
        for i in (seq (count $argv))
            if [ $cmd[$i] != $argv[$i] ]
                return 1
            end
        end
        return 0
    end
    return 1
end

function __fish_pack_packages
    pack list | string split ' =>' --field 1
end

complete -c pack -n "__fish_using_command pack" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack" -f -a "list" -d 'List installed packages'
complete -c pack -n "__fish_using_command pack" -f -a "install" -d 'Install new packages/plugins'
complete -c pack -n "__fish_using_command pack" -f -a "uninstall" -d 'Uninstall packages/plugins'
complete -c pack -n "__fish_using_command pack" -f -a "config" -d 'Configure/edit the package specific configuration'
complete -c pack -n "__fish_using_command pack" -f -a "move" -d 'Move a package to a different category or make it optional.'
complete -c pack -n "__fish_using_command pack" -f -a "update" -d 'Update packages'
complete -c pack -n "__fish_using_command pack" -f -a "generate" -d 'Generate the pack package file'
complete -c pack -n "__fish_using_command pack" -f -a "completions" -d 'Generates completion scripts for your shell'
complete -c pack -n "__fish_using_command pack" -f -a "help" -d 'Prints this message or the help of the given subcommand(s)'
complete -c pack -n "__fish_using_command pack list" -s c -l category -d 'List packages under this category'
complete -c pack -n "__fish_using_command pack list" -s s -l start -d 'List start packages'
complete -c pack -n "__fish_using_command pack list" -s o -l opt -d 'List optional packages'
complete -c pack -n "__fish_using_command pack list" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack list" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack install" -s c -l category -d 'Install package under provided category'
complete -c pack -n "__fish_using_command pack install" -l on -d 'Command for loading the plugins'
complete -c pack -n "__fish_using_command pack install" -l for -d 'Load this plugins for specific types'
complete -c pack -n "__fish_using_command pack install" -l build -d 'Build command for build package'
complete -c pack -n "__fish_using_command pack install" -s j -l threads -d 'Installing packages concurrently'
complete -c pack -n "__fish_using_command pack install" -s o -l opt -d 'Install plugins as opt(ional)'
complete -c pack -n "__fish_using_command pack install" -s l -l local -d 'Install local plugins'
complete -c pack -n "__fish_using_command pack install" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack install" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack uninstall" -s a -l all -d 'remove all package related configuration as well'
complete -c pack -n "__fish_using_command pack uninstall" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack uninstall" -f -a "(__fish_pack_packages)"
complete -c pack -n "__fish_using_command pack uninstall" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack config" -s d -l delete -d 'Delete package configuration file'
complete -c pack -n "__fish_using_command pack config" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack config" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack config" -f -a "(__fish_pack_packages)"
complete -c pack -n "__fish_using_command pack move" -s o -l opt -d 'Make package optional'
complete -c pack -n "__fish_using_command pack move" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack move" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack move" -f -a "(__fish_pack_packages)"
complete -c pack -n "__fish_using_command pack update" -s s -l skip -d 'Skip packages'
complete -c pack -n "__fish_using_command pack update" -s p -l packfile -d 'Regenerate the \'_pack\' file (combine all package configurations)'
complete -c pack -n "__fish_using_command pack update" -s j -l threads -d 'Updating packages concurrently'
complete -c pack -n "__fish_using_command pack update" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack update" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack update" -f -a "(__fish_pack_packages)"
complete -c pack -n "__fish_using_command pack generate" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack generate" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack completions" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack completions" -s V -l version -d 'Prints version information'
complete -c pack -n "__fish_using_command pack help" -s h -l help -d 'Prints help information'
complete -c pack -n "__fish_using_command pack help" -s V -l version -d 'Prints version information'
