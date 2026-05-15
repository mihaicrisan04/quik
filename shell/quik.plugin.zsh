# quik — inline AI prompt for your zsh session.
# https://github.com/mihaicrisan04/quik
#
# Bind a key (default ^_ aka Cmd+Enter via Ghostty's `text:0x1f`) to pop up an
# input bar, send the question through the `quik` binary, and stream the
# response in place. Per-shell session state lets follow-ups remember earlier
# turns, and recent shell commands are forwarded as context.
#
# Configuration (set before sourcing this file):
#   QUIK_BACKEND     claude | codex | auto       (default: auto)
#   QUIK_KEYBIND     bindkey escape sequence     (default: '^_')
#   QUIK_BIN         path to the quik binary     (default: quik on PATH)
#   QUIK_WIDTH       wrap column for responses   (default: min(COLUMNS, 90))

: ${QUIK_BACKEND:=auto}
: ${QUIK_KEYBIND:=^_}
: ${QUIK_BIN:=quik}

# Per-shell state. One claude session id (so follow-ups in the same terminal
# share context) + a small ring of recently-run shell commands (with exit
# codes) to forward as context on the next ask. Both reset when a new shell
# starts.
typeset -g  _QUIK_SESSION_ID="${_QUIK_SESSION_ID:-$(uuidgen 2>/dev/null | tr '[:upper:]' '[:lower:]')}"
typeset -gi _QUIK_SESSION_USED=${_QUIK_SESSION_USED:-0}
typeset -ga _QUIK_CMDS_PENDING
typeset -g  _QUIK_LAST_CMD=""

_quik_pre()  { _QUIK_LAST_CMD="$1" }
_quik_post() {
  local rc=$?
  [[ -z "$_QUIK_LAST_CMD" ]] && return
  # Mask values for keys that look like secrets before they reach the model.
  local line="${_QUIK_LAST_CMD//$'\n'/ }"
  line=$(printf '%s' "$line" | sed -E 's/(token|key|secret|password|passwd|pwd|auth|bearer)([= ])[^[:space:]]+/\1\2***/gi')
  _QUIK_CMDS_PENDING+=("[$rc] $line")
  (( ${#_QUIK_CMDS_PENDING} > 30 )) && _QUIK_CMDS_PENDING=("${_QUIK_CMDS_PENDING[@]: -30}")
  _QUIK_LAST_CMD=""
}

[[ -z "${preexec_functions[(r)_quik_pre]}"  ]] && preexec_functions+=(_quik_pre)
[[ -z "${precmd_functions[(r)_quik_post]}" ]] && precmd_functions+=(_quik_post)

# Shimmer animation: moving bright wave across "⠋ thinking...". The leading
# cell cycles through the cli-spinners classic "dots" braille frames, and the
# wave colorizes every cell including the spinner.
_quik_shimmer() {
  emulate -L zsh
  local suffix="thinking..."
  local -a frames
  frames=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
  local nframes=${#frames}
  local len=$((${#suffix} + 2))
  local pos=0 frame=1 j dist b ch
  printf '\033[?25l'
  while :; do
    printf '\r'
    for ((j=1; j<=len; j++)); do
      (( dist = pos - j ))
      (( dist < 0 )) && (( dist = -dist ))
      case $dist in
        0) b=255 ;;
        1) b=215 ;;
        2) b=170 ;;
        3) b=130 ;;
        *) b=90 ;;
      esac
      if (( j == 1 )); then
        ch="${frames[frame]}"
      elif (( j == 2 )); then
        ch=" "
      else
        ch="${suffix[j-2]}"
      fi
      printf '\033[38;2;%d;%d;%dm%s' $b $b $b "$ch"
    done
    printf '\033[0m'
    (( pos = (pos + 1) % (len + 4) ))
    (( frame = (frame % nframes) + 1 ))
    sleep 0.06
  done
}

quik-ask() {
  emulate -L zsh
  setopt local_options no_monitor no_notify
  local q="$*"
  [[ -z "$q" ]] && return

  local width=${QUIK_WIDTH:-$((COLUMNS < 90 ? COLUMNS : 90))}

  # Prepend any commands run since the last ask so the model has shell context.
  if (( ${#_QUIK_CMDS_PENDING} > 0 )); then
    q=$'Recent shell commands (format "[exit_code] command"):\n'"${(F)_QUIK_CMDS_PENDING}"$'\n\n'"$q"
    _QUIK_CMDS_PENDING=()
  fi

  # First call mints the session; later calls resume it for continuous context.
  local -a session_flags
  if [[ -n "$_QUIK_SESSION_ID" ]]; then
    if (( _QUIK_SESSION_USED == 0 )); then
      session_flags=(--session-id "$_QUIK_SESSION_ID")
    else
      session_flags=(--session-id "$_QUIK_SESSION_ID" --resume)
    fi
  fi

  _quik_shimmer &
  local shimmer_pid=$!

  "$QUIK_BIN" stream \
    --backend "$QUIK_BACKEND" \
    --width "$width" \
    --shimmer-pid "$shimmer_pid" \
    "${session_flags[@]}" \
    "$q"
  local rc=$?

  # The binary kills the shimmer on first token, but make sure it's gone.
  kill -9 $shimmer_pid 2>/dev/null
  wait $shimmer_pid 2>/dev/null

  if (( rc != 0 )); then
    print -P "%F{red}(no response from $QUIK_BACKEND)%f"
    return 1
  fi
  _QUIK_SESSION_USED=1
  print
}

# Pure-zsh single-line input that renders to /dev/tty and does NOT clear its
# rendering on exit (unlike gum/bubbletea). On Enter the typed line stays in
# place, so there's no visible blank frame between submission and what we
# print next. Args: $1=initial value (seed). On Enter: REPLY=value, returns 0.
# On Esc / Ctrl-C / empty submit: clears the input line and returns 1.
_quik_input() {
  emulate -L zsh
  local buf="${1:-}"
  local -i cursor=${#buf}
  local -i cur_row=0
  local C_RESET=$'\033[0m'
  local C_GREEN=$'\033[32m'
  local C_DIM_ITALIC=$'\033[3;38;5;240m'
  local PROMPT="${C_GREEN}❯${C_RESET} "
  local PLACEHOLDER="ask anything…"
  local -i PROMPT_W=2

  # Multi-row aware redraw. The previous render may have wrapped onto several
  # terminal rows; we move back up to the prompt's origin, clear from there to
  # end of screen, reprint, then reposition the cursor using wrap math.
  _quik_input_draw() {
    local -i cols=${COLUMNS:-80}
    (( cur_row > 0 )) && printf '\033[%dA' "$cur_row" >/dev/tty
    printf '\r\033[J' >/dev/tty

    local -i end_pos target_pos
    if [[ -z "$buf" ]]; then
      printf '%s%s%s%s' "$PROMPT" "$C_DIM_ITALIC" "$PLACEHOLDER" "$C_RESET" >/dev/tty
      end_pos=$((PROMPT_W + ${#PLACEHOLDER}))
      target_pos=$PROMPT_W
    else
      printf '%s%s' "$PROMPT" "$buf" >/dev/tty
      end_pos=$((PROMPT_W + ${#buf}))
      target_pos=$((PROMPT_W + cursor))
    fi

    # Auto-wrap quirk: when the rendered line ends exactly on a column
    # boundary, the terminal parks the cursor in "pending wrap" state on the
    # previous row instead of moving it down. Force the wrap with a sentinel
    # space, then erase it, so subsequent \033[A moves work from the row our
    # end_row math actually assumes.
    (( end_pos > 0 && end_pos % cols == 0 )) && printf ' \r\033[K' >/dev/tty

    local -i end_row=$((end_pos / cols))
    local -i target_row=$((target_pos / cols))
    local -i target_col=$((target_pos % cols))

    (( end_row > target_row )) && printf '\033[%dA' $((end_row - target_row)) >/dev/tty
    printf '\r' >/dev/tty
    (( target_col > 0 )) && printf '\033[%dC' "$target_col" >/dev/tty

    cur_row=$target_row
  }

  _quik_input_clear() {
    (( cur_row > 0 )) && printf '\033[%dA' "$cur_row" >/dev/tty
    printf '\r\033[J' >/dev/tty
    cur_row=0
  }

  _quik_input_draw

  local ch ch2 ch3 extra
  while IFS= read -rsk 1 ch </dev/tty; do
    case "$ch" in
      $'\r'|$'\n')
        if [[ -z "$buf" ]]; then
          _quik_input_clear
          return 1
        fi
        # Move cursor to end of buf (across wrapped rows), then newline.
        local -i cols=${COLUMNS:-80}
        local -i end_abs=$((PROMPT_W + ${#buf}))
        local -i end_row=$((end_abs / cols))
        local -i end_col=$((end_abs % cols))
        (( end_row > cur_row )) && printf '\033[%dB' $((end_row - cur_row)) >/dev/tty
        printf '\r' >/dev/tty
        (( end_col > 0 )) && printf '\033[%dC' "$end_col" >/dev/tty
        printf '\n' >/dev/tty
        REPLY="$buf"
        return 0
        ;;
      $'\x1b')
        if IFS= read -rsk 1 -t 0.05 ch2 </dev/tty 2>/dev/null; then
          case "$ch2" in
            '[')
              IFS= read -rsk 1 ch3 </dev/tty || ch3=""
              case "$ch3" in
                C) (( cursor < ${#buf} )) && { ((cursor++)); _quik_input_draw; } ;;
                D) (( cursor > 0 )) && { ((cursor--)); _quik_input_draw; } ;;
                H) (( cursor != 0 )) && { cursor=0; _quik_input_draw; } ;;
                F) (( cursor != ${#buf} )) && { cursor=${#buf}; _quik_input_draw; } ;;
                [0-9])
                  while IFS= read -rsk 1 -t 0.01 extra </dev/tty 2>/dev/null; do
                    [[ "$extra" == '~' ]] && break
                  done
                  ;;
              esac
              ;;
            $'\x7f'|$'\x08')
              # Option+Backspace → delete word backward
              local -i end_pos=$cursor
              while (( cursor > 0 )) && [[ "${buf:$((cursor-1)):1}" == ' ' ]]; do ((cursor--)); done
              while (( cursor > 0 )) && [[ "${buf:$((cursor-1)):1}" != ' ' ]]; do ((cursor--)); done
              buf="${buf:0:$cursor}${buf:$end_pos}"
              _quik_input_draw
              ;;
          esac
        else
          # Standalone Esc → cancel
          _quik_input_clear
          return 1
        fi
        ;;
      $'\x7f'|$'\x08')
        if (( cursor > 0 )); then
          buf="${buf:0:$((cursor-1))}${buf:$cursor}"
          ((cursor--))
          _quik_input_draw
        fi
        ;;
      $'\x03')
        _quik_input_clear
        return 1
        ;;
      $'\x15')
        if [[ -n "$buf" ]]; then buf=""; cursor=0; _quik_input_draw; fi
        ;;
      $'\x17')
        local -i end_pos=$cursor
        while (( cursor > 0 )) && [[ "${buf:$((cursor-1)):1}" == ' ' ]]; do ((cursor--)); done
        while (( cursor > 0 )) && [[ "${buf:$((cursor-1)):1}" != ' ' ]]; do ((cursor--)); done
        buf="${buf:0:$cursor}${buf:$end_pos}"
        _quik_input_draw
        ;;
      $'\x01')
        (( cursor != 0 )) && { cursor=0; _quik_input_draw; }
        ;;
      $'\x05')
        (( cursor != ${#buf} )) && { cursor=${#buf}; _quik_input_draw; }
        ;;
      *)
        if [[ -n "$ch" ]]; then
          buf="${buf:0:$cursor}${ch}${buf:$cursor}"
          ((cursor++))
          _quik_input_draw
        fi
        ;;
    esac
  done
  _quik_input_clear
  return 1
}

quik-prompt-widget() {
  local seed="$BUFFER"
  BUFFER=""
  zle reset-prompt

  if ! _quik_input "$seed"; then
    zle reset-prompt
    return
  fi
  local q="$REPLY"

  quik-ask "$q"
  print
  zle reset-prompt
}
zle -N quik-prompt-widget
bindkey "$QUIK_KEYBIND" quik-prompt-widget
