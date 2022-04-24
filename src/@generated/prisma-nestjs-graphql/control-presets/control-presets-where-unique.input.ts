import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ControlPresetsMonitorIdPresetCompoundUniqueInput } from './control-presets-monitor-id-preset-compound-unique.input';

@InputType()
export class ControlPresetsWhereUniqueInput {

    @Field(() => ControlPresetsMonitorIdPresetCompoundUniqueInput, {nullable:true})
    MonitorId_Preset?: ControlPresetsMonitorIdPresetCompoundUniqueInput;
}
