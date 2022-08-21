import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { DevicesCreateManyInput } from './devices-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyDevicesArgs {

    @Field(() => [DevicesCreateManyInput], {nullable:false})
    @Type(() => DevicesCreateManyInput)
    data!: Array<DevicesCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
