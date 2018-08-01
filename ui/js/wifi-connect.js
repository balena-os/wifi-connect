$('.chosen-select').chosen({disable_search_threshold: 10});

$(document).on('change', '.custom-file-input', function () {
	let fileName = $(this).val().replace(/\\/g, '/').replace(/.*\//, '');
	$(this).parent('.custom-file').find('.custom-file-label').text(fileName);
});

$("#settings").hide();
//$("#wifi-networks").hide();

$(function(){

	$('#wifi-networks-list a').on('click', function (e) {
		e.preventDefault();

		$("#wifi-networks").hide();
		$("#settings").show();
	})

	$(".navbar-brand").click(function (arg) {
		console.log(arg);
    });

	$('#security-select').chosen().change(function (event) {
		$("option:selected", this).tab('show');
	});

	$('#eap-authentication').chosen().change(function (event) {
		$("option:selected", this).tab('show');

		if($(this).val() === 'tls') {
			$("#eap-username-password-pane").hide();
		} else {
			$("#eap-username-password-pane").show();
		}
	});

	var networks = undefined;

	function showHideEnterpriseSettings() {
		var security = $(this).find(':selected').attr('data-security');
		if(security === 'eap') {
			$('#identity-group').show();
		} else {
			$('#identity-group').hide();
		}
	}

	$('#ssid-select').change(showHideEnterpriseSettings);

	$('#connect-form').submit(function(ev){
		$.post('/connect', $('#connect-form').serialize(), function(data){
			$('.before-submit').hide();
			$('#submit-message').removeClass('hidden');
		});
		ev.preventDefault();
	});

	$.get('/networks', function(data){
		if(data.length === 0){
			$('.before-submit').hide();
			$('#no-networks-message').removeClass('hidden');
		} else {
			networks = JSON.parse(data);
			$.each(networks, function(i, val){
				$('#ssid-select').append(
					$('<option>')
						.text(val.ssid)
						.attr('val', val.ssid)
						.attr('data-security', val.security)
				);
			});

			jQuery.proxy(showHideEnterpriseSettings, $('#ssid-select'))();
		}
	});
});
