import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusUpdateManyMutationInput } from '../monitor-status/monitor-status-update-many-mutation.input';
import { Type } from 'class-transformer';
import { Monitor_StatusWhereInput } from '../monitor-status/monitor-status-where.input';

@ArgsType()
export class UpdateManyMonitorStatusArgs {

    @Field(() => Monitor_StatusUpdateManyMutationInput, {nullable:false})
    @Type(() => Monitor_StatusUpdateManyMutationInput)
    data!: Monitor_StatusUpdateManyMutationInput;

    @Field(() => Monitor_StatusWhereInput, {nullable:true})
    @Type(() => Monitor_StatusWhereInput)
    where?: Monitor_StatusWhereInput;
}
