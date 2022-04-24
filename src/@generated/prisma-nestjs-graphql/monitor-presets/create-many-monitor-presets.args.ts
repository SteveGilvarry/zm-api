import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorPresetsCreateManyInput } from './monitor-presets-create-many.input';

@ArgsType()
export class CreateManyMonitorPresetsArgs {

    @Field(() => [MonitorPresetsCreateManyInput], {nullable:false})
    data!: Array<MonitorPresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
