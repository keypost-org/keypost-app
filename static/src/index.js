import _ from 'lodash';
import './css/app.css';

function init() {
}

function component() {
    const element = document.createElement('h3');
    element.innerHTML = _.join(['See', 'what', 'we\'re', 'up', 'to', 'at', '<a href="https://github.com/keypost-org">@keypost-org</a>'], ' ');
    return element;
  }

init();
document.body.appendChild(component());