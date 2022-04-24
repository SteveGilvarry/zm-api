import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsCreateInput } from './monitor-presets-create.input';

@ArgsType()
export class CreateOneMonitorPresetsArgs {

    @Field(() => MonitorPresetsCreateInput, {nullable:false})
    data!: MonitorPresetsCreateInput;
}
