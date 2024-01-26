#!/bin/bash

errno_command_failed=1    # command to execute
errno_unexpected_input=2  # unexpected input or script bug
errno_unexpected_exit=3   # maybe script bug

echo_and_exit () {
    if [[ $# == 0 ]]; then
        echo 'echo_and_exit: need more than a argument'
        exit $errno_unexpected_exit
    fi

    echo "$1: exit at l.${BASH_LINENO[0]} " >&2

    if [[ $# -ge $2 ]]; then
        exit $2
    else
        exit $errno_unexpected_exit
    fi
}

cd $(dirname $(cd $(dirname $0) && pwd))

args=()
is_raw_command=
while [[ $# != 0 ]]; do
    case $1 in
        -r | --raw-command ) is_raw_command=yes;;
        *                  ) args+=("$1");;
    esac
    shift
done

if [[ ${#args[@]} -eq 0 ]]; then
    echo_and_exit 'command is no specified' $errno_unexpected_input
fi

exec_commands=()
if [[ $is_raw_command == yes ]]; then
    exec_commands+=("${args[0]}")
else
    cmds=()
    case ${args[0]} in
        fmt    ) cmds+=('fmt');;
        clippy ) cmds+=('clippy');;
        build  ) cmds+=('build');;
        test   ) cmds+=('test');;
        all    ) cmds+=('fmt' 'clippy' 'build' 'test');;
        *      ) echo_and_exit "command '${args[0]}' is not registered" $errno_unexpected_input;;
    esac
    for cmd in ${cmds[@]}; do
        case $cmd in
            fmt    ) exec_commands+=('cargo fmt --all -- --check');;
            clippy ) exec_commands+=('cargo clippy --all-targets --all-features');;
            build  ) exec_commands+=('cargo build --verbose');;
            test   ) exec_commands+=('cargo test --verbose');;
            *      ) echo_and_exit "command '$cmd' is not defined";;
        esac
    done
fi

if [[ $(git status --porcelain --untracked-files=no | wc -l) -gt 0 ]]; then
    is_dirty_tree=yes
    echo "WARNING: the current working tree is dirty"
    echo "Execute command only to the current working"
else
    is_dirty_tree=no
fi

current_wd='current'
revs=(${args[@]:1:((${#args[@]}-1))})
if [[ $is_dirty_tree = yes ]]; then
    shas=($current_wd)
elif [[ ${#revs[@]} -eq 0 ]] || \
     ( [[ ${#revs[@]} -eq 1 ]] && ! [[ ${revs[0]} =~ (.*\.\..*) ]] ); then
    if [[ ${#revs[@]} -eq 0 ]] || \
       ( [[ ${#revs[@]} -eq 1 ]] && [[ ${revs[0]} == 'HEAD' ]] ); then
        branch_name=$(git branch --show-current)
    else
        branch_name=${revs[0]}
    fi
    head_name=$branch_name

    if [[ -z $branch_name ]]; then
        # get the default branch
        branch_name=$(git remote show $remote_name | grep 'HEAD branch' | awk '{print $NF}') || \
                    echo_and_exit 'fail to get the default branch'
        head_name='HEAD'
    fi

    remote_name=$(git remote show | head -n 1) || echo_and_exit 'fail to get the remote name'
    before_sha=$(git rev-parse $remote_name/$branch_name 2> /dev/null)
    ret=$?
    if [[ $ret -ne 0 ]]; then
        # get the default branch
        branch_name=$(git remote show $remote_name | grep 'HEAD branch' | awk '{print $NF}') || \
                    echo_and_exit 'fail to get the default branch'
        before_sha=$(git rev-parse $remote_name/$branch_name) || \
                   echo_and_exit 'fail to get start point sha'
    fi

    echo "commits to check: $before_sha..$head_name"
    shas=$(git rev-list $before_sha..$head_name) || echo_and_exit 'fail to get commit list'
elif [[ ${#revs[@]} -eq 2 ]]; then
    echo "commits to check: ${revs[0]}..${revs[1]}"
    shas=$(git rev-list ${revs[0]}..${revs[1]}) \
         || echo_and_exit 'fail to get commit list' $errno_unexpected_input
else
    echo "commits to check: ${revs[@]}"
    shas=$(git rev-list ${revs[@]}) || echo_and_exit 'fail to get commit list' $errno_unexpected_input
fi

# reverse the list of commit id
shas=($(echo $shas | tr -s ' ' '\n' | tac))

dirty_test_and_checkout () {
    if [[ $is_dirty_tree = no ]]; then
        git checkout -q $1
    fi
}
head_ref=$(git branch --show-current)
if [[ -z $head_ref ]]; then
    head_ref=$(git rev-parse HEAD)
fi
restore_head () {
    dirty_test_and_checkout $head_ref
}
restore_head_and_exit () {
    restore_head
    if [[ $# != 0 ]]; then
        exit $1
    else
        exit $errno_command_failed
    fi
}

for sha in ${shas[@]}; do
    decration_len=80
    header="### [$sha] ###"
    header=$header$(yes "#" | head -n $(($decration_len - ${#header})) | tr -d '\n')
    echo $header
    for cmd in "${exec_commands[@]}"; do
        if [[ $sha != $current_wd ]]; then
            dirty_test_and_checkout $sha || restore_head_and_exit $errno_unexpected_exit
        fi
        echo "# $cmd"
        $cmd
        ret=$?
        if [[ $ret -eq 0 ]]; then
            echo "OK"
        else
            echo "[$sha] command '$cmd': NG" >&2
            restore_head_and_exit
        fi
    done
    echo $(yes "#" | head -n $decration_len | tr -d '\n')
    echo
done

echo "Chacker passed!!!"
restore_head
