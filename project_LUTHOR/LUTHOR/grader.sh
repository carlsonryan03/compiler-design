#!/usr/bin/env bash

# unlike SIM projects, each compiler project has a name of its own
PROGRAM=LUTHOR

# this shifts off ${1:-.} to COMPLOC
source "${COMPGRADING}/comp-lib.sh"
source "${GRADERDIR}/grader-exec.sh"

set -e

# FIXME:  routines should rm -f _nooutput* before execution,
#         check that _nooutput* are all empty (we permit that they may be created)

# make sure this script is executable
test -x "${graderloc}/normoutput" || chmod +x "${graderloc}/normoutput"

# create a temporary dir to hold basicio tests in GRADERLOC, this permits multiple 
# simultaneous grading sessions without stumbling over each other...
GRADERBIO="_luthortest"
mkdir -p "${GRADERBIO}"
mirror "${graderloc}/basicio" "${GRADERBIO}"
# -i.bak compatible between bsd and linux --- but 2021 spring students complained of 
# permissions errors with this operation (from bash -x logging) --- so I'm just 
# squashing the errors into /dev/null 
sed 2>/dev/null -i.bak -e "s#_basicio#${GRADERBIO}#" "${GRADERBIO}/_abc.u" "${GRADERBIO}/_badtt.u"

NOTOKEN=_test_empty.token_output_file
# RUBRIC 1 ###
tallydir=`create_tally_dir 1 MAX=5 Inaccessible file names or paths`
# inaccessible scan.u, program.src
tally_missing_datafile "${tallydir}/scanu" "${comploc}/${PROGRAM}" MISSINGDATAFILE "${GRADERBIO}/_abc.src" ${NOTOKEN} 
tally_missing_datafile "${tallydir}/src" "${comploc}/${PROGRAM}" "${GRADERBIO}/_abc.u" MISSINGDATAFILE ${NOTOKEN} 

# inaccessible output location
rm -rf "${graderloc}/${GRADERBIO}/_dne"
tally_nooutput_exitnonzero "${tallydir}/dneoutput" "${comploc}/${PROGRAM}" "${GRADERBIO}/_abc.u" "${GRADERBIO}/_abc.src" "${GRADERBIO}/_dne/_output.tok" 

# inaccessible transition table entry in scanner definition entry
tally_nooutput_exitnonzero "${tallydir}/invalidtt" "${comploc}/${PROGRAM}" "${GRADERBIO}/_badtt.u" "${GRADERBIO}/_abc.src" ${NOTOKEN} 

# RUBRIC 2 ###
tallydir=`create_tally_dir 2 MAX=5 Missing data "(empty scan.u)"`
# empty scanner definition file
tally_nooutput_exitnonzero "${tallydir}/scanu" "${comploc}/${PROGRAM}" EMPTYDATAFILE "${GRADERBIO}/_abc.src" ${NOTOKEN} 

# RUBRIC 3 ###
tallydir=`create_tally_dir 3 MAX=5 Empty program.src`
# empty source files should exit 0 and truncate a pre-existing output file
tally_nooutput_exitzero "${tallydir}/src" "${comploc}/${PROGRAM}" "${GRADERBIO}/_abc.u" EMPTYDATAFILE ${NOTOKEN} 

# RUBRIC 4 ###
tallydir=`create_tally_dir 4 MAX=5 Invalid character input`
# invalid character in source file
tally_ignore_exitnonzero "${tallydir}/xyz" "${comploc}/${PROGRAM}" "${GRADERBIO}/_abc.u" "${GRADERBIO}/_xyz.src" "_output.tok" 

rm -rf "${GRADERBIO}"

test_token_source()
{
	# $1=graderloc tokenset dir
	# $2=tally prefix for .tokgood .tokbad
	# $3=graderloc tokenset dir source  (.src)
	local tally="${2}"
	local src="${3}"
	# tokens are either named after the program, or they are called tokens.dat
	local tok="${3/.src/.tok}"
	test -s "${graderloc}/${1}/${tok}" || tok=tokens.dat
	
	local testdir=_luthortest
	mkdir -p "${testdir}/${1}"
	mirror "${graderloc}/${1}" "${testdir}/${1}/"
	rm -f "${testdir}/${1}/${tok}"   # lest it be looked up in a side-channel
	# fix up path to .tt files
	sed 2>/dev/null -i.bak -e '/\.tt/s#^#'"${testdir}/#" "${testdir}/${1}/scan.u"
	
	rm -f ./_output.tok
	if test_run "${comploc}/${PROGRAM}" "${testdir}/${1}/scan.u" "${testdir}/${1}/${3}" ./_output.tok ; then
		touch "${tally}.bad"
	fi
	if ! test -s ./_output.tok ; then
		grader_echo "ERROR: No ./_output.tok generated."
		touch "${tally}.bad"
	fi
	# no point in continuing
	test -f "${tally}.bad" && return 0

	"${graderloc}/normoutput" < ./_output.tok  >./_students.tok
	"${graderloc}/normoutput" < "${graderloc}/${1}/${tok}" >./_expected.tok
	if ! diff _students.tok _expected.tok >/dev/null ; then 
		touch "${tally}.bad"
		grader_msg <<EOT
Tokenization for scanner definition ${1}/scan.u and source ${1}/${src}
in '${graderloc}' FAILED.  The output from your ${PROGRAM} is in _output.tok.
_output.tok was normalized to _students.tok and compared to _expected.tok.

You can see which lines differ using "visual diff" tool, or at the console with

  $ diff -u _students.tok _expected.tok

(- lines would be removed from _students.tok and + lines would be added to 
match _expected.tok.)

IF YOU WANT TO INSPECT THIS FAILURE, CTRL-C now!
EOT
		grader_keystroke
	else 
		touch "${tally}.good"
		grader_echo "Tokenization for scanner definition ${1}/scan.u and source ${1}/${src} in '${graderloc}' is GOOD :)"
		rm -rf "${testdir}"
	fi
}

test_token_set()
{
	# $1=graderloc tokenset dir
	# $2=tally prefix for the tokenset
	for src in `( cd "${graderloc}/${1}" && ls -1d *.src )` ; do 
		test_token_source "${1}" "${2}/${1/\//-}.${src/.src/}" "${src}"
	done
}


test_category()
{
	# $1=rubric line item, $2=category key, $3... description
	local rli=${1}
	local cat=${2}
	local mx=${3}
	shift 3
	local td=`create_tally_dir ${rli} ${mx} "${*}"`
	for ts in `( cd "${graderloc}" && ls -1d ${cat}/? )` ; do 
		test_token_set ${ts} ${td}
	done
}

test_category 5 tied MAX=10 "Tied matches over one line"
test_category 6 disjoint MAX=10 "Disjoint matches over one line"
test_category 7 overnl MAX=10 "Matches over multiple lines"
test_category 8 complex MAX=10 "Complex matches"

show_tallies

# always show cwd at the end, so grader is sure the correct results
# are recorded for the correct submission (the upload id is in the path)
pwd

