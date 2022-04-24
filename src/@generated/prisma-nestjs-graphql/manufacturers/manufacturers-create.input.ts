import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';

@InputType()
export class ManufacturersCreateInput {

    @Field(() => String, {nullable:false})
    Name!: string;
}
