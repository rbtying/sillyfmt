import * as wasm from "sillyfmt-wasm";



var text_input = document.querySelector("#text-input");
var text_output = document.querySelector("#text-output");

function debounce(func, wait, immediate) {
	var timeout;
	return function() {
		var context = this, args = arguments;
		var later = function() {
			timeout = null;
			if (!immediate) func.apply(context, args);
		};
		var callNow = immediate && !timeout;
		clearTimeout(timeout);
		timeout = setTimeout(later, wait);
		if (callNow) func.apply(context, args);
	};
};

function format(unformatted) {
    const formatted = wasm.format(unformatted);
    text_output.innerText = formatted;
}

var debounced_format = debounce(format, 16, false);

text_input.addEventListener('input', function(event) {
    const unformatted = event.target.value;
    debounced_format(unformatted);
});
