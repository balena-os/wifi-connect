$(function(){
	var networks = undefined;

	function showHideEnterpriseSettings() {
		var security = $(this).find(':selected').attr('data-security');
		if(security === 'enterprise') {
			$('#identity-group').show();
		} else {
			$('#identity-group').hide();
		}
	}

	$('#ssid-select').change(showHideEnterpriseSettings);

	$.get("/networks", function(data){
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

	$('#connect-form').submit(function(ev){
		$.post('/connect', $('#connect-form').serialize(), function(data){
			$('.before-submit').hide();
			$('#submit-message').removeClass('hidden');
		});
		ev.preventDefault();
	});
});
