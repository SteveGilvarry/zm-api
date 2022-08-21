import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusWhereUniqueInput } from '../monitor-status/monitor-status-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneMonitorStatusArgs {

    @Field(() => Monitor_StatusWhereUniqueInput, {nullable:false})
    @Type(() => Monitor_StatusWhereUniqueInput)
    where!: Monitor_StatusWhereUniqueInput;
}
