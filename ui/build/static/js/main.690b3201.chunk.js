(this["webpackJsonpwifi-connect-ui"]=this["webpackJsonpwifi-connect-ui"]||[]).push([[0],{1186:function(e,t,n){"use strict";n.r(t);var i=n(0),a=n.n(i),s=n(79),o=n.n(s),r=(n(480),n(481),n(408)),l=n(409),c=n.n(l),u=n(1199),m=n(1197),d=n(465),f=n(22),p=n(63),_=n(1198);const w={en:{welcome_message:"Wifi Connect. Please choose your WiFi from the list"},fr:{welcome_message:"Wifi Connect. S\xe9lectionnez le r\xe9seau Wifi \xe0 connecter dans la liste"}},h={en:{welcome_message:"Please choose your WiFi from the list",passphrase:"Passphrase",ssid:"Wifi network name (SSID)",connect_wifi:"Connect",no_wifi_network_available:"No wifi networks available.",ensure_wifi_in_range:"Please ensure there is a network within range and reboot the device.",applying_changes:"Applying changes",failed_to_fetch_wifi_networks:"Failed to fetch available networks",failed_to_connect_to_wifi_networks:"Failed to connect to the network.",select_ssid:"Select SSID",device_will_soon_be_online:"Your device will soon be online. If connection is unsuccessful, the Access Point will be back up in a few minutes, and reloading this page will allow you to try again."},fr:{welcome_message:"S\xe9lectionnez le r\xe9seau Wifi \xe0 connecter dans la liste",passphrase:"Mot de passe Wifi",ssid:"R\xe9seau Wifi (SSID)",connect_wifi:"Se connecter",no_wifi_network_available:"Aucun r\xe9seau Wifi n'est visible.",ensure_wifi_in_range:"Assurez-vous qu'un r\xe9seau Wifi est proche et red\xe9marrez votre p\xe9riph\xe9rique.",applying_changes:"Changement en cours",failed_to_fetch_wifi_networks:"Impossible d'identifier les r\xe9seaux Wifi",failed_to_connect_to_wifi_networks:"Impossible de se connecter au r\xe9seau Wifi.",select_ssid:"S\xe9lectionner le r\xe9seau Wifi",device_will_soon_be_online:"Votre p\xe9riph\xe9rique devrait bient\xf4t \xeatre en ligne. Si la connexion ne fonctionne pas, le portail sera de retour dans quelques minutes, vous pourrez alors recharger cette page et r\xe9essayer. Si ce n'est pas le cas, red\xe9marrez votre p\xe9riph\xe9rique."}};function g(e){let t=function(){let e="";switch(!0){case!!window.navigator.language:e=window.navigator.language;break;default:e="en"}return 2!==e.length&&(e=e.substring(0,2)),e in h?e:"en"}();return t in w&&e in w[t]?w[t][e]:t in h&&e in h[t]?h[t][e]:e in w.en?w.en[e]:e in h.en?h.en[e]:""}const b=e=>{var t;return{type:"object",properties:{ssid:{title:g("ssid"),type:"string",default:null===(t=e[0])||void 0===t?void 0:t.ssid,oneOf:e.map(e=>({const:e.ssid,title:e.ssid}))},identity:{title:"User",type:"string",default:""},passphrase:{title:g("passphrase"),type:"string",default:""}},required:["ssid"]}},v=e=>{let{availableNetworks:t,onSubmit:n}=e;const[a,s]=i.useState({}),o=(r=t,l=a.ssid,r.some(e=>e.ssid===l&&"enterprise"===e.security));var r,l,c;return i.createElement(f.a,{flexDirection:"column",alignItems:"center",justifyContent:"center",m:4,mt:5},i.createElement(p.a.h3,{align:"center",mb:4},g("welcome_message")),i.createElement(_.a,{width:["100%","80%","60%","40%"],onFormChange:e=>{let{formData:t}=e;s(t)},onFormSubmit:e=>{let{formData:t}=e;return n(t)},value:a,schema:b(t),uiSchema:(c=o,{ssid:{"ui:placeholder":g("select_ssid"),"ui:options":{emphasized:!0}},identity:{"ui:options":{emphasized:!0},"ui:widget":c?void 0:"hidden"},passphrase:{"ui:widget":"password","ui:options":{emphasized:!0}}}),submitButtonProps:{width:"60%",mx:"20%",mt:3,disabled:t.length<=0},submitButtonText:g("connect_wifi")}))};var y=n(467),S=n(15);const k=e=>{let{hasAvailableNetworks:t,attemptedConnect:n,error:a}=e;return i.createElement(i.Fragment,null,n&&i.createElement(y.a,{m:2,info:!0},i.createElement(S.a.span,null,g("applying_changes")),i.createElement(S.a.span,null,g("device_will_soon_be_online"))),!t&&i.createElement(y.a,{m:2,warning:!0},i.createElement(S.a.span,null,g("no_wifi_network_available"),"\xa0"),i.createElement(S.a.span,null,g("ensure_wifi_in_range"))),!!a&&i.createElement(y.a,{m:2,danger:!0},i.createElement(S.a.span,null,a)))};var E,W=n(2);const C=Object(W.createGlobalStyle)(E||(E=Object(r.a)(["\n\tbody {\n\t\tmargin: 0;\n\t\tfont-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',\n\t\t\t'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',\n\t\t\tsans-serif;\n\t\t-webkit-font-smoothing: antialiased;\n\t\t-moz-osx-font-smoothing: grayscale;\n\t}\n\n\tcode {\n\t\tfont-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace;\n\t}\n"])));var x=()=>{const[e,t]=a.a.useState(!1),[n,i]=a.a.useState(!0),[s,o]=a.a.useState(""),[r,l]=a.a.useState([]);a.a.useEffect(()=>{fetch("/networks").then(e=>{if(200!==e.status)throw new Error(e.statusText);return e.json()}).then(l).catch(e=>{o("".concat(g("failed_to_fetch_wifi_networks"),". ").concat(e.message||e))}).finally(()=>{i(!1)})},[]);return a.a.createElement(u.a,null,a.a.createElement(C,null),a.a.createElement(m.a,{style:{backgroundColor:""},brand:a.a.createElement("img",{src:c.a,style:{height:30},alt:"logo"})}),a.a.createElement(d.a,null,a.a.createElement(k,{attemptedConnect:e,hasAvailableNetworks:n||r.length>0,error:s}),a.a.createElement(v,{availableNetworks:r,onSubmit:e=>{t(!0),o(""),fetch("/connect",{method:"POST",body:JSON.stringify(e),headers:{"Content-Type":"application/json"}}).then(e=>{if(200!==e.status)throw new Error(e.statusText)}).catch(e=>{o("".concat(g("failed_to_connect_to_wifi_networks")," ").concat(e.message||e))})}})))};o.a.render(a.a.createElement(x,null),document.getElementById("root"))},409:function(e,t,n){e.exports=n.p+"static/media/logo.34c0c94e.svg"},475:function(e,t,n){e.exports=n(1186)}},[[475,1,2]]]);