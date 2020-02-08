echo '_TaaS_autocomplete() {
	local autoc
	autoc=$(ls ./scenarios)
	if [ -z "$autoc" ]
	then
		echo;	
	else
		autoc="$autoc del"
		autoc=$(echo $autoc | sed "s/common_data //")
	fi
	cur="${COMP_WORDS[COMP_CWORD]}"
    	COMPREPLY=( $(compgen -W "$autoc" -- ${cur}) )	
	return 0;
}' > ~/.taasfunctions.bashrc

sed -i '/. ~\/.taasfunctions.bashrc/d' ~/.bashrc #delete of line below already exists
echo '. ~/.taasfunctions.bashrc' >> ~/.bashrc
sed -i '/complete -F _TaaS_autocomplete .\/orchestrator/d' ~/.bashrc #delete of line below already exists
echo 'complete -F _TaaS_autocomplete ./orchestrator' >> ~/.bashrc

source ~/.bashrc