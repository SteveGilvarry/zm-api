import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';

@ArgsType()
export class FindUniqueMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:false})
    where!: Monitor_StatusWhereUniqueInput;
}
