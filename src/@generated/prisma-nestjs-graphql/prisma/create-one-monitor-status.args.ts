import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusCreateInput } from '../monitor-status/monitor-status-create.input';

@ArgsType()
export class CreateOneMonitorStatusArgs {

    @Field(() => Monitor_StatusCreateInput, {nullable:false})
    data!: Monitor_StatusCreateInput;
}
