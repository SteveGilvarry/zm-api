import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Monitor_StatusCreateManyInput } from '../monitor-status/monitor-status-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyMonitorStatusArgs {

    @Field(() => [Monitor_StatusCreateManyInput], {nullable:false})
    @Type(() => Monitor_StatusCreateManyInput)
    data!: Array<Monitor_StatusCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
