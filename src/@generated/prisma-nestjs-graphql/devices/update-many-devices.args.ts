import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesUpdateManyMutationInput } from './devices-update-many-mutation.input';
import { Type } from 'class-transformer';
import { DevicesWhereInput } from './devices-where.input';

@ArgsType()
export class UpdateManyDevicesArgs {

    @Field(() => DevicesUpdateManyMutationInput, {nullable:false})
    @Type(() => DevicesUpdateManyMutationInput)
    data!: DevicesUpdateManyMutationInput;

    @Field(() => DevicesWhereInput, {nullable:true})
    @Type(() => DevicesWhereInput)
    where?: DevicesWhereInput;
}
